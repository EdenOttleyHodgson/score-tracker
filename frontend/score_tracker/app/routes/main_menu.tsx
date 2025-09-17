import {
  useContext,
  useState,
  type Dispatch,
  type FormEvent,
  type SetStateAction,
} from "react";
import type { Route } from "./+types/main_menu";
import { BackendConnection, useBackendHook } from "~/backend";
import type { ServerMessage } from "~/backend/types";

// provides `loaderData` to the component
export async function loader({ params }: Route.LoaderArgs) {
  return {};
}

export default function MainMenu() {
  // const [name, setName] = useState("")
  // const [code, setCode] = useState("")
  //
  // function handleSubmit(data: FormData) {
  //   console.log(data.get("name"), data.get("roomCode"));
  // }
  return (
    <form action="/room">
      <label>
        Display Name:
        <input name="name" defaultValue="User" />
      </label>
      <label>
        Room Code:
        <input name="roomCode" />
      </label>
      <button type="submit">Join</button>
    </form>
  );
}
