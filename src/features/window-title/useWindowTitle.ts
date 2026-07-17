import { useEffect, useState } from "react";

import { setWindowTitle } from "../../ipc/windowTitle";

export function useWindowTitle(displayName: string | null, unsaved: boolean) {
  const [feedback, setFeedback] = useState("");

  useEffect(() => {
    let current = true;
    void setWindowTitle({ displayName, unsaved }).then((result) => {
      if (current) {
        setFeedback(
          result.status === "applied" ? "" : "DRAFT could not update the window title.",
        );
      }
    });
    return () => {
      current = false;
    };
  }, [displayName, unsaved]);

  return feedback;
}
