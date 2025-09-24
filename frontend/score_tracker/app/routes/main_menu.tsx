import { useState } from "react";
import type { Route } from "./+types/main_menu";
import { NavLink, useOutletContext } from "react-router";
import type { LayoutContext } from "./layout";

// provides `loaderData` to the component
export async function loader({ params }: Route.LoaderArgs) {
  return {};
}

export default function MainMenu() {
  const { adminPass } = useOutletContext<LayoutContext>();
  const [roomCode, setRoomCode] = useState("");
  const [displayName, setDisplayName] = useState("");
  return (
    <div>
      <label>
        Display Name:
        <input name="name" onChange={(e) => setDisplayName(e.target.value)} />
      </label>
      <label>
        Room Code:
        <input
          name="roomCode"
          value={roomCode}
          onChange={(e) => setRoomCode(e.target.value)}
        />
      </label>
      <label>
        Admin Password:
        <input
          name="Admin Password"
          value={adminPass.value}
          onChange={(e) => adminPass.setter(e.target.value)}
        />
      </label>
      <NavLink to={`room/${roomCode}?name=${displayName}`}>Join</NavLink>
      <NavLink to={`room/${roomCode}?create=true&name=${displayName}`}>
        Create
      </NavLink>
    </div>
  );
}
