import { useNavigate } from "react-router";
import { useWebSocket } from "react-use-websocket/dist/lib/use-websocket";
import { SOCKET_URL, useBackendSocket } from "~/backend";

export async function loader() {
  return {};
}

export default function NoServerConnection() {
  const nav = useNavigate();
  useWebSocket(SOCKET_URL, {
    onOpen: () => {
      nav("/");
    },
    reconnectAttempts: Number.MAX_SAFE_INTEGER,
  });
  return <p>Cannot connect to server</p>;
}
