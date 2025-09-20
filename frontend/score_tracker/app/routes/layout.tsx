import React, { createContext, useState } from "react";
import { Outlet, useNavigate } from "react-router";
import { useBackendSocket } from "~/backend";

export async function loader() { }

export type LayoutContext = {
  adminPass: {
    value: string;
    setter: React.Dispatch<React.SetStateAction<string>>;
  };
  displayName: {
    value: string;
    setter: React.Dispatch<React.SetStateAction<string>>;
  };
};
export default function Layout() {
  const displayName = useState("User");
  const adminPass = useState("");
  const navigator = useNavigate();
  const [socket, sendMessage] = useBackendSocket(
    (msg) => {
      console.log(msg);
    },
    (e) => {
      console.error(e);
      navigator("./noServerConnection");
    }
  );
  const context: LayoutContext = {
    displayName: { value: displayName[0], setter: displayName[1] },
    adminPass: { value: adminPass[0], setter: adminPass[1] },
  };
  return <Outlet context={context}></Outlet>;
}
