import { useEffect } from "react";
import type { ServerMessage, ClientMessage } from "./types";
import * as typia from "typia";

type MessageHook = (arg0: ServerMessage) => void;

export class BackendConnection {
  private socket: WebSocket;
  private static instance: BackendConnection | null = null;
  private message_hooks: Map<string, MessageHook> = new Map();

  static getInstance(): BackendConnection {
    if (!BackendConnection.instance) {
      BackendConnection.instance = new BackendConnection(
        import.meta.env.VITE_BACKEND_SOCKET_URL
      );
    }
    return BackendConnection.instance;
  }
  private constructor(uri: string) {
    this.socket = new WebSocket(uri);
    this.socket.addEventListener("message", (ev) => {
      let parsed = parse_message(ev.data);
      if (parsed) {
        this.message_hooks.forEach((hook) => hook(parsed));
      }
    });
  }

  add_hook(id: string, f: MessageHook) {
    this.message_hooks.set(id, f);
  }
  remove_hook(id: string) {
    this.message_hooks.delete(id);
  }
  // set_on_message(f: (s: ServerMessage) => void) {
  //   this.socket.addEventListener("message", (msg) => {
  //     let parsed = parse_message(msg);
  //     if (parsed) {
  //       f(parsed);
  //     }
  //   });
  // }
  send_message(msg: ClientMessage) {
    console.log("Sending msg");
    this.socket.send(JSON.stringify(msg));
  }
}

export function useBackendHook(id: string, hook: MessageHook) {
  useEffect(() => {
    BackendConnection.getInstance().add_hook(id, hook);
    return () => {
      BackendConnection.getInstance().remove_hook(id);
    };
  }, []);
}

function parse_message(msg: string): ServerMessage | null {
  try {
    return typia.json.assertParse<ServerMessage>(msg);
  } catch (e) {
    console.error(e);
    return null;
  }
}
