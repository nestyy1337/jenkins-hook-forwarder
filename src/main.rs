use jenkins_hooks::App;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt().init();

    let app = App::create_app().await;
    app.run().await;
}
