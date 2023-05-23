use neon::{prelude::*, types::Deferred};
use std::{sync::mpsc, thread, time};

struct Wt {
    tx: mpsc::Sender<WtMessage>,
}

type WtCallback = Box<dyn FnOnce(&Channel, Deferred) + Send>;

enum WtMessage {
    Callback(Deferred, WtCallback),
    Close,
}

impl Finalize for Wt {}

impl Wt {
    fn new<'a, C>(cx: &mut C) -> Result<Self, String>
    where
        C: Context<'a>,
    {
        // create the channel for sending callbacks to execute on the thread
        let (tx, rx) = mpsc::channel::<WtMessage>();

        // create a 'Channel' for callin back to JavaScript.
        let channel = cx.channel();

        // spawn a thread for handling async functions
        thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                match msg {
                    WtMessage::Callback(deferred, f) => {
                        f(&channel, deferred);
                    }
                    WtMessage::Close => break,
                }
            }
        });

        Ok(Self { tx })
    }

    fn close(&self) -> Result<(), mpsc::SendError<WtMessage>> {
        self.tx.send(WtMessage::Close)
    }

    fn send(
        &self,
        def: Deferred,
        cb: impl FnOnce(&Channel, Deferred) + Send + 'static,
    ) -> Result<(), mpsc::SendError<WtMessage>> {
        self.tx.send(WtMessage::Callback(def, Box::new(cb)))
    }
}

impl Wt {
    fn js_new(mut cx: FunctionContext) -> JsResult<JsBox<Wt>> {
        let wt = Wt::new(&mut cx).or_else(|err| cx.throw_error(err.to_string()))?;
        Ok(cx.boxed(wt))
    }

    fn js_get_ready(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let wt = cx.this().downcast_or_throw::<JsBox<Wt>, _>(&mut cx)?;
        let (deferred, promise) = cx.promise();

        wt.send(deferred, move |channel, deferred| {
            // let's stall for 5 seconds
            thread::sleep(time::Duration::from_secs(5));

            // JS thread execute
            deferred.settle_with(channel, move |mut cx| Ok(cx.null()));
        })
        .into_rejection(&mut cx)?;

        Ok(promise)
    }

    fn js_get_stats(mut cx: FunctionContext) -> JsResult<JsPromise> {
        let wt = cx.this().downcast_or_throw::<JsBox<Wt>, _>(&mut cx)?;
        let (deferred, promise) = cx.promise();

        wt.send(deferred, move |channel, deferred| {
            // do nothing on our thread for now

            // JS thread execute
            deferred.settle_with(channel, move |mut cx| Ok(cx.null()));
        })
        .into_rejection(&mut cx)?;

        Ok(promise)
    }

    fn js_close(mut cx: FunctionContext) -> JsResult<JsUndefined> {
        cx.this()
            .downcast_or_throw::<JsBox<Wt>, _>(&mut cx)?
            .close()
            .or_else(|err| cx.throw_error(err.to_string()))?;
        Ok(cx.undefined())
    }
}

trait SendResultExt {
    fn into_rejection<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<()>;
}

impl SendResultExt for Result<(), mpsc::SendError<WtMessage>> {
    fn into_rejection<'a, C: Context<'a>>(self, cx: &mut C) -> NeonResult<()> {
        self.or_else(|err| {
            let msg = err.to_string();
            match err.0 {
                WtMessage::Callback(deferred, _) => {
                    let err = cx.error(msg)?;
                    deferred.reject(cx, err);
                    Ok(())
                }
                WtMessage::Close => cx.throw_error("Expected WtMessage::Callback"),
            }
        })
    }
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("wtNew", Wt::js_new)?;
    cx.export_function("wtGetReady", Wt::js_get_ready)?;
    cx.export_function("wtGetStats", Wt::js_get_stats)?;
    cx.export_function("wtClose", Wt::js_close)?;
    Ok(())
}
