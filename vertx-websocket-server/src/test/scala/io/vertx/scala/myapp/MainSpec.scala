package io.vertx.scala.myapp

import java.util.concurrent.Executors

import io.vertx.scala.core._
import org.scalatest._

import scala.concurrent.{Await, ExecutionContext, Future}
import scala.concurrent.duration._

class MainSpec extends AsyncFlatSpec with BeforeAndAfter with Matchers {
  val vertx = Vertx.vertx()

  "Websocket" should "produce a valid JsonObject" in{
    implicit val context = ExecutionContext.fromExecutor(Executors.newSingleThreadExecutor())
    val f = Future {
      Main.main(Array())
    }

    val client = vertx.createHttpClient()

    val wsResult = client.webSocketFuture(8666, "localhost", "/").flatMap(ws => {
      val promise = scala.concurrent.Promise[String]()
      ws.handler(b => {
        val jsonObject = b.toJsonObject
        if (jsonObject.containsKey("time") && jsonObject.containsKey("value")) {
          promise.success("All matched")
        } else {
          promise.failure(new Exception)
        }
        ws.close()
      })
      promise.future
    })

    val str = Await.result(wsResult, 2500 millis)
    str shouldBe "All matched"
  }
}
