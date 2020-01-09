package io.vertx.scala.myapp

import io.vertx.ext.web.handler.StaticHandler
import io.vertx.lang.scala.VertxExecutionContext
import io.vertx.scala.core._
import io.vertx.scala.ext.web.Router

import scala.util.{Failure, Success}

object Main {
  def main(args: Array[String]): Unit = {
    val vertx = Vertx.vertx()
    implicit val exc:VertxExecutionContext = VertxExecutionContext(vertx.getOrCreateContext())

    val router = Router.router(vertx)
    router
      .get("/hello")
      .handler(_.response().end("world"))

    router
      .route("/static/*")
      .handler(StaticHandler.create().setCachingEnabled(false))

    var ct = 0;

    val eventualServer = vertx
      .createHttpServer()
      .requestHandler(router)
      .webSocketHandler(ctx => {
        vertx.setPeriodic(1000, _ => {
          ct = ct + 1
          ctx.writeTextMessage(JsonObject().put("value", ct).encode())
        })
      })
      .listenFuture(8666, "0.0.0.0")

    eventualServer.onComplete {
      case Success(_) => println("Done")
      case Failure(t) => println("An error has occurred: " + t.getMessage)
    }
  }
}
