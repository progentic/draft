import { invokeCommand } from "./client";
import { isRecord } from "./documentEnvelope";

export type TextAnalysisFindingCode = keyof typeof FINDING_POLICIES;
export type TextAnalysisCategory = "clarity" | "cohesion" | "grammar" | "tone" | "voice";
export type TextAnalysisSeverity = "advice" | "warning";

export interface TextAnalysisFinding {
  code: TextAnalysisFindingCode;
  category: TextAnalysisCategory;
  severity: TextAnalysisSeverity;
  startByte: number;
  endByte: number;
  title: string;
  explanation: string;
}

export type TextAnalysisErrorCode =
  | "cancelled"
  | "empty_text"
  | "helper_failed"
  | "invalid_output"
  | "runtime_unavailable"
  | "text_too_long"
  | "timed_out"
  | "worker_unavailable";

export type TextAnalysisClientError =
  | { type: "command"; code: TextAnalysisErrorCode }
  | { type: "invalid-response" }
  | { type: "transport" };

export type TextAnalysisResult =
  | { status: "ready"; findings: TextAnalysisFinding[] }
  | { status: "error"; error: TextAnalysisClientError };

export const FINDING_POLICIES = {
  repeated_word: {
    category: "grammar",
    severity: "warning",
    title: "Repeated word",
    explanation: "An adjacent word may have been duplicated. Review the repetition before editing.",
  },
  long_sentence: {
    category: "clarity",
    severity: "advice",
    title: "Long sentence",
    explanation: "This sentence contains more than 30 words. Consider whether smaller parts would be easier to follow.",
  },
  all_caps_emphasis: {
    category: "tone",
    severity: "advice",
    title: "Extended capital emphasis",
    explanation: "A word of five or more letters uses all capitals and may read as unusually forceful.",
  },
  repeated_sentence_opener: {
    category: "cohesion",
    severity: "advice",
    title: "Repeated sentence opening",
    explanation: "Consecutive sentences begin with the same substantial word and may feel repetitive.",
  },
  mixed_first_person: {
    category: "voice",
    severity: "advice",
    title: "First-person perspective shift",
    explanation: "Both singular and plural first-person pronouns appear. Review whether the perspective shift is intentional.",
  },
} as const;

const COMMAND_NAME = "run_text_analysis";

export async function runTextAnalysis(text: string): Promise<TextAnalysisResult> {
  try {
    const response = await invokeCommand<unknown>(COMMAND_NAME, { request: { text } });
    return findingsResult(response);
  } catch (error: unknown) {
    return { status: "error", error: analysisError(error) };
  }
}

function findingsResult(response: unknown): TextAnalysisResult {
  if (
    isRecord(response) &&
    Object.keys(response).length === 1 &&
    isRecord(response.result) &&
    Object.keys(response.result).length === 1 &&
    Array.isArray(response.result.findings) &&
    response.result.findings.every(isTextAnalysisFinding)
  ) {
    return { status: "ready", findings: response.result.findings };
  }
  return { status: "error", error: { type: "invalid-response" } };
}

function analysisError(error: unknown): TextAnalysisClientError {
  if (isTextAnalysisError(error)) {
    return { type: "command", code: error.code };
  }
  return { type: "transport" };
}

function isTextAnalysisFinding(value: unknown): value is TextAnalysisFinding {
  if (!isRecord(value) || !isFindingCode(value.code)) {
    return false;
  }
  const policy = FINDING_POLICIES[value.code];
  return (
    Object.keys(value).length === 7 &&
    value.category === policy.category &&
    value.severity === policy.severity &&
    value.title === policy.title &&
    value.explanation === policy.explanation &&
    isByteRange(value.startByte, value.endByte)
  );
}

function isFindingCode(value: unknown): value is TextAnalysisFindingCode {
  return typeof value === "string" && Object.hasOwn(FINDING_POLICIES, value);
}

function isByteRange(start: unknown, end: unknown) {
  return (
    Number.isSafeInteger(start) &&
    Number.isSafeInteger(end) &&
    Number(start) >= 0 &&
    Number(end) > Number(start)
  );
}

function isTextAnalysisError(value: unknown): value is { code: TextAnalysisErrorCode } {
  return (
    isRecord(value) &&
    Object.keys(value).length === 1 &&
    (value.code === "cancelled" ||
      value.code === "empty_text" ||
      value.code === "helper_failed" ||
      value.code === "invalid_output" ||
      value.code === "runtime_unavailable" ||
      value.code === "text_too_long" ||
      value.code === "timed_out" ||
      value.code === "worker_unavailable")
  );
}
