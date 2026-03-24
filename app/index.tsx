import { invoke } from "@tauri-apps/api/core";

import { useEffect, useState } from "react";
import {
  BaseForm,
  ScrollView,
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
  const [exchange, setExchange] = useState<Record<string, unknown>[]>([]);
  const [activeConversation, setActiveConversation] = useState<string>("");

  useEffect(() => {
    dispatch("get_exchanges", {}).then(data => {
      const exchanges = [...Object.values(data.exchanges)];
      exchanges.sort((a, b) => ((a as { createTime: number }).createTime) - ((b as { createTime: number }).createTime));
      console.log(exchanges);
    });
  }, []);

  return (
    <View
      style={{
        flex: 1,
      }}
      p={20}
    >
      {exchange.length > 0 ? exchange.map((item, index) => (
        <ScrollView key={index} style={{ flex: 1 }}>
          <Text maxWidth="75%" backgroundColor="$gray2" py={5} px={10} mt={5} borderRadius={10} alignSelf="flex-end">{(item?.prompt as string)?.trim()}</Text>
          <Text maxWidth="75%" backgroundColor="$gray4" py={5} px={10} mt={5} borderRadius={10} alignSelf="flex-start">{(item?.response as string)?.trim()}</Text>
        </ScrollView>
      )) : <View style={{ flex: 1 }} />}
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
            setExchange([...exchange, {...data, response: "..."}]);
            dispatch("form_submitted", data).then(data => setExchange((prev) => [...prev.slice(0, -1), data]));
          }
        }
      />
    </View>
  );
}
