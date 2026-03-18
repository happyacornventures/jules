import {
  DarkTheme,
  ThemeProvider
} from "@react-navigation/native";
import { Stack } from "expo-router";
import { TamaguiProvider, Theme } from "tamagui";

import { config } from "@tamagui/config";
import { createTamagui } from "tamagui";

const tamaguiConfig = createTamagui(config);

type Conf = typeof tamaguiConfig;

declare module "tamagui" {
  interface TamaguiCustomConfig extends Conf {}
}

export default function RootLayout() {
  return (
    <TamaguiProvider config={tamaguiConfig} defaultTheme="dark">
      <ThemeProvider value={DarkTheme}>
        <Theme name="dark">
          <Stack screenOptions={{ headerShown: false }} />
        </Theme>
      </ThemeProvider>
    </TamaguiProvider>
  );
}
