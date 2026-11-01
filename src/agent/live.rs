//! Replies being generated on this instance, with every SSE frame sent so far, so a client that
//! reloads or navigates back can reconnect (`GET /api/agent/chat/{id}/stream`) and replay the
//! reply from its start, then follow it live.

use futures_util::Stream;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

pub struct LiveReply {
  /// Cancels the reply (Stop on this instance; other instances poll the DB flag).
  pub cancel: CancellationToken,
  frames: Mutex<Vec<Arc<str>>>,
  done: AtomicBool,
  /// Bumped on every new frame and at the end.
  tick: watch::Sender<()>,
}

impl LiveReply {
  pub fn new(cancel: CancellationToken) -> Arc<Self> {
    Arc::new(LiveReply {
      cancel,
      frames: Mutex::new(Vec::new()),
      done: AtomicBool::new(false),
      tick: watch::channel(()).0,
    })
  }

  pub fn push(&self, frame: String) {
    self
      .frames
      .lock()
      .unwrap_or_else(|p| p.into_inner())
      .push(frame.into());
    self.tick.send_replace(());
  }

  /// No more frames; subscribers end once they have read everything.
  pub fn finish(&self) {
    self.done.store(true, Ordering::Release);
    self.tick.send_replace(());
  }

  /// Every frame from the first, then new ones as they come, until [`Self::finish`].
  pub fn subscribe(live: Arc<Self>) -> impl Stream<Item = Arc<str>> + Send + 'static {
    let rx = live.tick.subscribe();
    futures_util::stream::unfold((live, 0usize, rx), |(live, next, mut rx)| async move {
      loop {
        // Mark the current tick seen *before* reading, so a frame pushed after the read wakes
        // the wait below.
        rx.borrow_and_update();
        let frame = live
          .frames
          .lock()
          .unwrap_or_else(|p| p.into_inner())
          .get(next)
          .cloned();
        if let Some(frame) = frame {
          return Some((frame, (live, next + 1, rx)));
        }
        if live.done.load(Ordering::Acquire) || rx.changed().await.is_err() {
          return None;
        }
      }
    })
  }
}

static RUNNING: LazyLock<Mutex<HashMap<String, Arc<LiveReply>>>> = LazyLock::new(Default::default);

fn running() -> std::sync::MutexGuard<'static, HashMap<String, Arc<LiveReply>>> {
  RUNNING.lock().unwrap_or_else(|p| p.into_inner())
}

pub fn register(chat_id: &str, live: Arc<LiveReply>) {
  running().insert(chat_id.to_string(), live);
}

pub fn get(chat_id: &str) -> Option<Arc<LiveReply>> {
  running().get(chat_id).cloned()
}

/// Forget the reply, unless a newer one for the chat has replaced it.
pub fn unregister(chat_id: &str, live: &Arc<LiveReply>) {
  let mut map = running();
  if map.get(chat_id).is_some_and(|l| Arc::ptr_eq(l, live)) {
    map.remove(chat_id);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use futures_util::StreamExt;

  #[tokio::test]
  async fn late_subscribers_replay_then_follow() {
    let live = LiveReply::new(CancellationToken::new());
    live.push("a".into());
    let early = LiveReply::subscribe(live.clone());
    live.push("b".into());
    let late = LiveReply::subscribe(live.clone());
    let writer = {
      let live = live.clone();
      tokio::spawn(async move {
        tokio::task::yield_now().await;
        live.push("c".into());
        live.finish();
      })
    };
    let (early, late): (Vec<_>, Vec<_>) = tokio::join!(early.collect(), late.collect());
    writer.await.unwrap();
    let text = |v: Vec<Arc<str>>| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    assert_eq!(text(early), ["a", "b", "c"]);
    assert_eq!(text(late), ["a", "b", "c"]);
  }
}
