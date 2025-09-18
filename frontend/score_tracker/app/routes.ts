import { type RouteConfig, route, index } from "@react-router/dev/routes";

export default [
  index("./routes/main_menu.tsx"),
  route("/room/:roomCode", "./routes/room.tsx"),
  // pattern ^           ^ module file
] satisfies RouteConfig;
