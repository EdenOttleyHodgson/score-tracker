import {
  type RouteConfig,
  route,
  index,
  layout,
} from "@react-router/dev/routes";

export default [
  layout("./routes/layout.tsx", [
    index("./routes/main_menu.tsx"),
    route("/room/:roomCode", "./routes/room.tsx"),
  ]),
  route("/noServerConnection", "./routes/no_server_connection.tsx"),
  // pattern ^           ^ module file
] satisfies RouteConfig;
