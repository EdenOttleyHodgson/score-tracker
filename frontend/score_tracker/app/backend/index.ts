import type { ServerMessage, ClientMessage } from "./types";
import * as typia from "typia";

export class BackendConnection {
  socket: WebSocket;
  constructor(uri: string) {
    this.socket = new WebSocket(uri);
  }
  set_on_message(f: (s: ServerMessage) => void) {
    this.socket.addEventListener("message", (msg) => {
      let parsed = parse_message(msg);
      if (parsed) {
        f(parsed);
      }
    });
  }
  send_message(msg: ClientMessage) {
    this.socket.send(JSON.stringify(msg));
  }
}

function parse_message(msg: MessageEvent<any>): ServerMessage | null {
  try {
    return typia.json.assertParse<ServerMessage>(msg.data);
  } catch (e) {
    console.error(e);
    return null;
  }
}
