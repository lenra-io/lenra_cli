import { DataApi } from "@lenra/app-server";
import { Flex, View } from "@lenra/components";
import { Counter } from "../classes/Counter.js";
import { views } from "../index.gen.js";

export default function (_data, _props) {
    return Flex([
        View(views.counter)
            .data(DataApi.collectionName(Counter), {
                "user": "@me"
            })
            .props({ text: "My personnal counter" }),
        View(views.counter)
            .data(DataApi.collectionName(Counter), {
                "user": "global"
            })
            .props({ text: "The common counter" }),
    ])
        .direction("vertical")
        .spacing(16)
        .mainAxisAlignment("spaceEvenly")
        .crossAxisAlignment("center")
}

