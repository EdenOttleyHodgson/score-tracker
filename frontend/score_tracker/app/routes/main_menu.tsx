import { useState } from "react";
import type { Route } from "./+types/main_menu";
import { NavLink } from "react-router";
import { BackendConnection } from "~/backend";

// provides `loaderData` to the component
export async function loader({ params }: Route.LoaderArgs) {
  return {};
}

export default function MainMenu() {
  const [roomCode, setRoomCode] = useState("")
  return (
    <div>
      <label>
        Display Name:
        <input name="name" onChange={e => localStorage.setItem("displayName", e.target.value)} />
      </label>
      <label>
        Room Code:
        <input name="roomCode" value={roomCode} onChange={e => setRoomCode(e.target.value)} />
      </label>
      <NavLink to={`room/${roomCode}`}>Join</NavLink>
      <NavLink to={`room/${roomCode}?create=true`}>Create</NavLink>


    </div>
  );
}
