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



    val eventualServer = vertx
      .createHttpServer()
      .requestHandler(router)
      .webSocketHandler(ctx => {
        var ct = 0;
        val r = scala.util.Random

        val periodicId = vertx.setPeriodic(100, _ => {
          ctx.writeTextMessage(JsonObject()
            .put("time", ct.toFloat)
            .put("value", r.between(-1.0f, 1.0f))
            .encode())
          ct += 1
        })

        ctx.closeHandler(c => {
          vertx.cancelTimer(periodicId);
        })

      })
      .listenFuture(8666, "0.0.0.0")

    eventualServer.onComplete {
      case Success(_) => println("Done")
      case Failure(t) => println("An error has occurred: " + t.getMessage)
    }
  }
}
