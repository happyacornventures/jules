import { invoke } from "@tauri-apps/api/core";

import {
  BaseForm,
  View
} from "./component";

// eventually this should be split based on platform as well--this is only true for web
const dispatch = (event: string, payload: Record<string, unknown>) =>
  window.__TAURI_INTERNALS__
    ? invoke("dispatch", {
        event,
        payload: JSON.stringify(payload),
      }).then((rsp) => JSON.parse(rsp as string))
    : Promise.resolve({ node: {} });

export default function Index() {
  return (
    <View
      style={{
        flex: 1,
      }}
    >
      <BaseForm
        schema={{
          title: "Prompt",
          properties: {
            prompt: {
              type: "string",
              title: "prompt",
              value: "",
            },
          },
        }}
        onSubmit={data => dispatch("form_submitted", data).then(console.log)}
      />
    </View>
  );
}
