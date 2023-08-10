import { Flex, View } from "@lenra/components";
import { views } from "../index.gen.js";

export default function (_data, _props) {
  return Flex([
    View(views.menu),
    View(views.home)
  ])
    .direction("vertical")
    .scroll(true)
    .spacing(4)
    .crossAxisAlignment("center")
}

