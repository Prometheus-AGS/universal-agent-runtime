import { useState } from "react";
import {
  setSessionModel,
  useSessionModelField,
} from "../../../platform/entities/session-configuration/domain";

export function SessionField() {
  const [open, setOpen] = useState(false);
  const field = useSessionModelField();

  return (
    <button
      type="button"
      onClick={() => {
        setOpen((value) => !value);
        setSessionModel("configured/model");
      }}
    >
      {open ? field.value : "Show model"}
    </button>
  );
}
