use tracing::info;
use socketioxide::extract::{Data, SocketRef}; 

pub async fn on_connect(s: SocketRef) {
    info!("socket connected: {}", s.id);

    s.on("join", |s: SocketRef, Data::<String>(room)| {
        info!("joined room {}", room);
        s.join(room)
        .ok();
    });

    s.on(
        "new message", 
        |s: SocketRef, Data::<String>(msg)| {
            println!("rooms connected: {:?}",s.rooms());
            s.broadcast().emit("new message", "hi")
            .ok();
        }
    );

    s.on(
        "typing", |s: SocketRef| {
            s.broadcast()
                .emit("typing", "typing")
                .ok();
        }
    );

    s.on_disconnect(|s: SocketRef| {
        s.broadcast().emit("user left", "gone")
        .ok();
    })

}