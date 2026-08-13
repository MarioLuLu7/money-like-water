import {
  Checkbox,
  Select,
  Switch,
  TextArea,
  TextField,
  type CheckboxProps,
  type SwitchProps,
  type TextAreaProps,
} from "@radix-ui/themes";
import type { ComponentProps } from "react";

type SelectOption = {
  value: string;
  label: string;
};

type FormInputProps = ComponentProps<typeof TextField.Root> & {
  className?: string;
};

type FormTextAreaProps = TextAreaProps & {
  className?: string;
};

type FormSelectProps = Omit<ComponentProps<typeof Select.Root>, "children"> & {
  options: SelectOption[];
  placeholder?: string;
  className?: string;
};

type FormSwitchProps = SwitchProps & {
  label?: string;
};

type FormCheckboxProps = CheckboxProps & {
  label?: string;
};

export function FormInput({ className, ...props }: FormInputProps) {
  return <TextField.Root className={className} size="2" variant="surface" {...props} />;
}

export function FormTextArea({ className, ...props }: FormTextAreaProps) {
  return <TextArea className={className} size="2" variant="surface" {...props} />;
}

export function FormSelect({ className, options, placeholder, ...props }: FormSelectProps) {
  return (
    <Select.Root {...props}>
      <Select.Trigger className={className} placeholder={placeholder} variant="surface" />
      <Select.Content position="popper" variant="solid">
        {options.map((option) => (
          <Select.Item key={option.value} value={option.value}>
            {option.label}
          </Select.Item>
        ))}
      </Select.Content>
    </Select.Root>
  );
}

export function FormSwitch({ label, ...props }: FormSwitchProps) {
  return (
    <span className="ui-toggle">
      <Switch size="1" {...props} />
      {label}
    </span>
  );
}

export function FormCheckbox({ label, ...props }: FormCheckboxProps) {
  return (
    <span className="ui-toggle">
      <Checkbox size="1" {...props} />
      {label}
    </span>
  );
}
