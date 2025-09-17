import {
  useContext,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import type { Route } from "./+types/main_menu";
import { BackendConnection } from "~/backend";
import type { ServerMessage } from "~/backend/types";

// provides `loaderData` to the component
export async function loader({ params }: Route.LoaderArgs) {
  return {};
}

export default function MainMenu() {
  const backend = BackendConnection.getInstance();
  const init_arr: ServerMessage[] = [];

  const [messages, setMessages] = useState(init_arr);
  backend.add_hook("main_menu", (msg) => {
    console.log(JSON.stringify(msg));
    setMessages((x) => [...x, msg]);
  });
  const messageList = messages.map((value, index) => (
    <li key={index}>{JSON.stringify(value)}</li>
  ));

  console.log(`yes....${backend}`);
  return (
    <div>
      <p>hello</p>
      <button
        onClick={() =>
          backend.send_message({
            kind: "JoinRoom",
            code: "AAAAAAAA",
            name: "my plarkatingas",
          })
        }
      >
        button
      </button>
      <ul>{messageList}</ul>
    </div>
  );
}
