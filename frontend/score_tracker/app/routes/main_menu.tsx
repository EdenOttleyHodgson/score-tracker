import { useContext } from "react";
import { BackendContext } from "~/root";
import type { Route } from "./+types/main_menu";

// provides `loaderData` to the component
export async function loader({ params }: Route.LoaderArgs) {
  return {};
}

export default function MainMenu() {
  const x = useContext(BackendContext);
  console.log(`yes....${x}`);
  return <p>hello</p>;
}
