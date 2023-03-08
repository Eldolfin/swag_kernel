use core::{pin::Pin, task::{Context, Poll}};

use crate::{eprintln, print};

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::{Stream, task::AtomicWaker, StreamExt};
use pc_keyboard::{Keyboard, ScancodeSet1, layouts, HandleControl, DecodedKey};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

/// The capacity of the SCANCODE_QUEUE. Should be changed in case typing
/// on the keyboard panics
const SCANCODE_QUEUE_CAPACITY: usize = 100;

static WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the keyboard interrup handler
///
/// Must not block or allocate to avoir deadlocks.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            panic!("Scancode queue full. Increase SCANCODE_QUEUE_CAPACITY");
        } else {
            WAKER.wake();
        }
    } else {
        eprintln!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    /// This field prevents the construction of the struct
    /// from outside this module to enfore the use of `new()`
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| 
            ArrayQueue::new(SCANCODE_QUEUE_CAPACITY))
        .expect("ScancodeStream::new should be only be called once");

        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // should never panic as the queue is initialized in the constructor
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("SCANCODE_QUEUE not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(), 
        layouts::Us104Key, 
        HandleControl::Ignore);
    
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(char) => print!("{char}"),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
