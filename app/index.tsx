import { invoke } from "@tauri-apps/api/core";

import { useState } from "react";
import {
  BaseForm,
  Text,
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
  const [exchange, setExchange] = useState<Record<string, unknown>>({});

  return (
    <View
      style={{
        flex: 1,
      }}
      p={20}
    >
      {Object.keys(exchange).length > 0 && (
        <>
          <Text maxWidth="75%" backgroundColor="$gray2" py={5} px={10} mt={5} borderRadius={10} alignSelf="flex-end">{(exchange?.prompt as string)?.trim()}</Text>
          <Text maxWidth="75%" backgroundColor="$gray4" py={5} px={10} mt={5} borderRadius={10} alignSelf="flex-start">{(exchange?.response as string)?.trim()}</Text>
        </>
      )}
      <BaseForm
        schema={{
          properties: {
            prompt: {
              type: "string",
              title: "prompt",
              value: "",
            },
          },
          display: "row",
          submitText: "Send"
        }}
        onSubmit={data => {
            setExchange({...data, response: "..."});
            dispatch("form_submitted", data).then(setExchange)
          }
        }
      />
    </View>
  );
}
