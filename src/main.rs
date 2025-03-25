use jenkins_hooks::{
    App,
    logging::{get_subscriber, init_subscriber},
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let defualt_subscriber = get_subscriber("INFO", "./log");
    init_subscriber(defualt_subscriber);

    let app = App::create_app().await;
    app.run().await;
}
