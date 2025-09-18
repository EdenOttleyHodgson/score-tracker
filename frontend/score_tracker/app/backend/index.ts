import { useEffect } from "react";
import type { ServerMessage, ClientMessage } from "./types";
import * as typia from "typia";

type MessageHook = (arg0: ServerMessage) => void;

export class BackendConnection {
  private socket: WebSocket;
  private static instance: BackendConnection | null = null;
  private message_hooks: Map<string, MessageHook> = new Map();
  static resetInstance() {
    BackendConnection.instance = null
  }

  static async getInstance(): Promise<BackendConnection> {
    if (BackendConnection.instance) {
      return BackendConnection.instance
    } else {
      return new Promise((resolve, reject) => {
        const instance = new BackendConnection(import.meta.env.VITE_BACKEND_SOCKET_URL, (_) => {
          BackendConnection.instance = instance
          resolve(BackendConnection.instance)
        }, (e) => {
          reject(e)
        })
      })
    }
  }

  private constructor(uri: string, on_open: (ev: Event) => void, on_error: (ev: Event) => void) {
    this.socket = new WebSocket(uri);
    this.socket.addEventListener("open", on_open)
    this.socket.addEventListener("error", on_error)
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

export function useBackendHook(id: string, hook: MessageHook, onFail: () => void) {
  useEffect(() => {
    BackendConnection.getInstance().then((instance) =>
      instance.add_hook(id, hook)
    ).catch((e) => { console.error(e); onFail() })


    return () => {
      BackendConnection.getInstance().then((instance) => instance.remove_hook(id)).catch((instance) => { BackendConnection.resetInstance(); onFail() })
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
