import { useEffect } from "react";
import type { ServerMessage, ClientMessage } from "./types";
import * as typia from "typia";
import { useWebSocket } from "react-use-websocket/dist/lib/use-websocket";

export function parseMessage(msg: string): ServerMessage | null {
  try {
    return typia.json.assertParse<ServerMessage>(msg);
  } catch (e) {
    console.error(e);
    return null;
  }
}

export function useBackendSocket(
  handler: (arg0: ServerMessage) => void,
  failHandler: (e: any) => void
): [any, (arg0: ClientMessage) => void] {
  const socket = useWebSocket(import.meta.env.VITE_BACKEND_SOCKET_URL, {
    onMessage: (ev) => {
      console.log("Recieved message", ev);
      const parsed = parseMessage(ev.data);
      if (parsed) {
        handler(parsed);
      } else {
        console.error("Could not parse message:", ev.data);
      }
    },
    onError: (ev) => {
      console.error("Web socket error:", ev);
      failHandler(ev);
    },
  });
  return [
    socket,
    (msg) => {
      socket.sendJsonMessage(msg);
    },
  ];
}
