pub mod pb {
    tonic::include_proto!("grpc.examples.echo");
}

use std::io;
use std::io::Write;
use futures::stream::Stream;
use std::time::Duration;
use futures::{TryStreamExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::StreamExt;
use tonic::transport::Channel;

use pb::{echo_client::EchoClient, EchoRequest};
use tonic::{IntoStreamingRequest, Streaming};

struct RawClient {
    service: EchoClient<Channel>,
}

struct SomeWrapper{
    tx: tokio::sync::mpsc::UnboundedSender<EchoRequest>,
    response_stream: Streaming<crate::pb::EchoResponse>,
}

impl SomeWrapper{
    pub async fn send(&mut self, message: String){
        self.tx.send(EchoRequest{message: "123".to_string()}).expect("TODO")
    }

    pub async fn receive(&mut self) -> Option<crate::pb::EchoResponse>{
        if let Some(receive_result) = self.response_stream.next().await {
            return Some(receive_result.unwrap());
        }
        return None;
    }
}

impl RawClient {
    pub async fn streaming_echo(&mut self) -> SomeWrapper {
        let (tx, rx): (tokio::sync::mpsc::UnboundedSender<EchoRequest>, tokio::sync::mpsc::UnboundedReceiver<EchoRequest>) = tokio::sync::mpsc::unbounded_channel();

        let request_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx).throttle(Duration::from_secs(1));

        let response_stream = self.service
            .bidirectional_streaming_echo(request_stream)
            .await
            .unwrap().into_inner();

        println!("Successfull init");
        return SomeWrapper{tx, response_stream};
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /*let send_fut = tokio::spawn(
         async move {
             let stdin = tokio::io::stdin();
             let mut reader = BufReader::new(stdin).lines();
             loop {
                 if let Ok(Some(line)) = reader.next_line().await {
                     tx.send(EchoRequest{message: line.clone()}).unwrap();
                 } else {
                     eprintln!("Ошибка чтения из stdin");
                     break;
                 }
             }
         }
     );*/ // reads lines from stdin and echoes


    let mut client = RawClient { service: EchoClient::connect("http://[::1]:50052").await.unwrap() };

    let mut wrapper = client.streaming_echo().await;
    //let fut = bidirectional_streaming_echo_throttle(client, Duration::from_secs(2));
    loop {
        wrapper.send("123".to_string()).await;
        if let Some(resp) = wrapper.receive().await{
            println!("{}", resp.message);
        }
    }
    //fut.await;
    //send_fut.await.unwrap();
    Ok(())
}
