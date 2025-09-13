import { type RouteConfig, route, index } from "@react-router/dev/routes";

export default [
  index("./routes/main_menu.tsx"),
  // pattern ^           ^ module file
] satisfies RouteConfig;
