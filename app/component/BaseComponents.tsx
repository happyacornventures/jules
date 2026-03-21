import { X } from "@tamagui/lucide-icons";
import { useEffect, useState } from "react";
import {
  Button,
  Form,
  H2,
  Input,
  Label,
  ListItem,
  Sheet,
  Text,
  View,
  XStack,
  YStack,
} from "tamagui";

const SearchableSelect = ({
  fieldKey,
  title,
  options,
  selection,
  value,
  handleChange,
}: {
  fieldKey: string;
  title: string;
  options: { label: string; value: unknown }[];
  selection: unknown[];
  value: unknown;
  handleChange: (key: string, value: unknown) => void;
}) => {
  const [query, setQuery] = useState(
    () => options.find((o) => o.value === value)?.label ?? "",
  );
  const [isFocused, setIsFocused] = useState(false);

  const filteredOptions =
    query && isFocused
      ? options.filter((option) =>
          option.label.toLowerCase().includes(query.toLowerCase()),
        )
      : [];

  return (
    <YStack gap="$2" marginVertical="$2">
      <Label htmlFor={fieldKey}>{title}</Label>
      <Input
        id={fieldKey}
        value={query}
        onChange={(e) => setQuery((e.currentTarget as HTMLInputElement).value)}
        onFocus={() => setIsFocused(true)}
        onBlur={() => setTimeout(() => setIsFocused(false), 150)} // Delay to allow click
        placeholder="Type to search..."
      />
      {filteredOptions.length > 0 && (
        <YStack borderTopWidth={1} borderColor="$borderColor">
          {filteredOptions.map((option) => (
            <ListItem
              hoverTheme
              pressTheme
              key={option.label}
              onPress={() => {
                handleChange(fieldKey, option.value);
                setQuery(option.label);
                setIsFocused(false);
              }}
            >
              {option.label}
            </ListItem>
          ))}
        </YStack>
      )}
    </YStack>
  );
};

const BaseFormField = ({
  fieldKey,
  title,
  type,
  value,
  handleChange,
}: {
  fieldKey: string;
  title: string;
  type: string;
  value: unknown;
  handleChange: (key: string, value: unknown) => void;
}) => (
  // <YStack key={fieldKey} gap="$2" marginVertical="$2">
  <>
    <Label htmlFor={fieldKey}>{title}</Label>
    <Input
      id={fieldKey}
      value={(value as string) || ""}
      onChange={(e) =>
        handleChange(fieldKey, (e.currentTarget as HTMLInputElement).value)
      }
      keyboardType={
        type === "number" || type === "integer" ? "numeric" : "default"
      }
    />
    </>
  // </YStack>
);

export const BaseForm = ({
  schema,
  onSubmit,
  onChange = () => {},
}: {
  schema: Record<string, unknown>;
  onSubmit: (values: Record<string, unknown>) => void;
  onChange?: (values: Record<string, unknown>) => void;
}) => {
  const [formData, setFormData] = useState<Record<string, unknown>>({});

  const handleChange = (key: string, value: unknown) => {
    setFormData((prev) => ({ ...prev, [key]: value }));
  };

  useEffect(() => {
    if (onChange) {
      onChange(formData);
    }
  }, [formData]);

  return (
    <>
      {schema.title && <H2>{schema.title}</H2>}
      <Form
        onSubmit={() => {
          onSubmit(formData);
          setFormData({});
        }}
      >
        {schema.properties &&
          Object.entries(schema.properties ?? {})
            .filter(([key, property]) => property.visible ?? true)
            .map(([key, property]) =>
              property.type === "search" ? (
                <SearchableSelect
                  key={key}
                  fieldKey={key}
                  title={property.title || key}
                  options={property.options || []}
                  selection={(formData[key] || []) as unknown[]}
                  value={formData[key] || property.value || ""}
                  handleChange={
                    property.values === "multiple"
                      ? (key, value) =>
                          handleChange(key, [
                            ...((formData[key] as unknown[]) || []),
                            value,
                          ])
                      : handleChange
                  }
                />
              ) : (
                <BaseFormField
                  key={key}
                  fieldKey={key}
                  title={property.title || key}
                  type={property.type || "text"}
                  value={formData[key] || property.value || ""}
                  handleChange={handleChange}
                />
              ),
            )}
        <Form.Trigger asChild>
          <Button marginTop="$4">{schema.submitText || "Submit"}</Button>
        </Form.Trigger>
      </Form>
    </>
  );
};

export const BottomDrawer = ({
  open,
  onOpenChange,
  children,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  children: React.ReactNode;
}) => (
  <Sheet
    modal
    open={open}
    onOpenChange={onOpenChange}
    snapPoints={[100]}
    position={0}
    onPositionChange={() => {}} // onPositionChange is required
    dismissOnSnapToBottom
  >
    <Sheet.Overlay />
    <Sheet.Frame ai="center" jc="center">
      <Sheet.Handle />
      <Button
        position="absolute"
        top="$3"
        right="$3"
        size="$2"
        circular
        icon={X}
        onPress={() => onOpenChange(false)}
      />
      {children}
    </Sheet.Frame>
  </Sheet>
);

const BaseListItem = ({
  node,
  targetKey,
}: {
  node: Record<string, unknown>;
  targetKey?: string;
}) => (
  <ListItem
    hoverTheme
    pressTheme
    key={node.id as string}
    onPress={(node.onPress as () => void) ?? (() => {})}
  >
    <XStack flex={1} justifyContent="space-between" alignItems="center">
      {targetKey && (
        <Text>{(node as Record<string, unknown>)?.[targetKey]}</Text>
      )}
      {node.actions &&
        Object.entries(node.actions ?? {}).map(([key, action]) => (
          <Button
            theme="red"
            size="$2"
            onPress={action as (e: unknown) => void}
          >
            {key}
          </Button>
        ))}
    </XStack>
  </ListItem>
);

export const BaseList = ({
  nodes,
  targetKey,
  ItemComponent = BaseListItem,
  ...styles
}: {
  nodes: Record<string, unknown>[];
  targetKey?: string;
  ItemComponent?: React.ComponentType<{
    node: Record<string, unknown>;
    targetKey?: string;
  }>;
} & Record<string, unknown>) => (
  <View {...styles}>
    {nodes.map((node) => (
      <ItemComponent
        key={node.id as string}
        node={node}
        targetKey={targetKey}
      />
    ))}
  </View>
);

export const BaseDetails = ({ node }: { node: Record<string, unknown> }) => {
  console.log("BaseDetails node:", node);

  return (
    <>
      {Object.entries(node ?? {}).filter(([key, value]) => value !== undefined && value !== null && typeof value === "string").map(([key, value]) => (
        <ListItem key={key}><Text>{`${key}: ${value}`}</Text></ListItem>
      ))}
    </>
  );
};
