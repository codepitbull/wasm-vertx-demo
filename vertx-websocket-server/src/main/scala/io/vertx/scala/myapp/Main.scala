package io.vertx.scala.myapp

import java.util

import io.vertx.core.http.impl.MimeMapping
import io.vertx.ext.web.handler.StaticHandler
import io.vertx.lang.scala.VertxExecutionContext
import io.vertx.scala.core._
import io.vertx.scala.ext.web.Router

import scala.util.{Failure, Success}

object Main {
  def main(args: Array[String]): Unit = {
    val vertx = Vertx.vertx()
    implicit val exc:VertxExecutionContext = VertxExecutionContext(vertx.getOrCreateContext())

    MimeMapping.getMimeTypeForExtension("hallo") //just make sure internal map is initialized
    val m = classOf[MimeMapping].getDeclaredField("m")
    m.setAccessible(true)

    val fieldValue: util.HashMap[java.lang.String, java.lang.String] = m.get(null).asInstanceOf[util.HashMap[java.lang.String, java.lang.String]]
    fieldValue.put("wasm", "application/wasm")

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
      .websocketHandler(ctx => {
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
