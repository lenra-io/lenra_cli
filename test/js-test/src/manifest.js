import { View } from "@lenra/components";
import { views } from "./index.gen.js";

export const lenra = {
    routes: [
        {
            path: "/",
            view: View(views.main)
        }
    ]
};