import type { Editor } from "@tiptap/react";

interface TextSegment {
  endByte: number;
  from: number;
  startByte: number;
  text: string;
}

export interface TextAnalysisSnapshot {
  excerpt: (startByte: number, endByte: number) => string;
  locate: (startByte: number, endByte: number) => { from: number; to: number } | null;
  text: string;
}

export function createTextAnalysisSnapshot(editor: Editor): TextAnalysisSnapshot {
  const segments: TextSegment[] = [];
  const parts: string[] = [];
  let bytes = 0;
  let previousEnd: number | null = null;

  editor.state.doc.descendants((node, position) => {
    if (!node.isText || !node.text) {
      return true;
    }
    if (previousEnd !== null && previousEnd !== position) {
      parts.push("\n");
      bytes += 1;
    }
    const encodedLength = utf8Length(node.text);
    segments.push({ startByte: bytes, endByte: bytes + encodedLength, from: position, text: node.text });
    parts.push(node.text);
    bytes += encodedLength;
    previousEnd = position + node.nodeSize;
    return true;
  });

  const text = parts.join("");
  return {
    text,
    excerpt: (startByte, endByte) => excerpt(text, startByte, endByte),
    locate: (startByte, endByte) => locateRange(segments, startByte, endByte),
  };
}

function locateRange(segments: TextSegment[], startByte: number, endByte: number) {
  const start = segments.find((segment) => segment.startByte <= startByte && startByte < segment.endByte);
  const end = [...segments].reverse().find((segment) => segment.startByte < endByte && endByte <= segment.endByte);
  if (!start || !end) {
    return null;
  }
  return {
    from: start.from + byteToCodeUnit(start.text, startByte - start.startByte),
    to: end.from + byteToCodeUnit(end.text, endByte - end.startByte),
  };
}

function excerpt(text: string, startByte: number, endByte: number) {
  const encoded = new TextEncoder().encode(text);
  const selected = new TextDecoder().decode(encoded.slice(startByte, endByte)).trim();
  return selected.length > 80 ? `${selected.slice(0, 77)}...` : selected;
}

function byteToCodeUnit(text: string, byteOffset: number) {
  let bytes = 0;
  let codeUnits = 0;
  for (const character of text) {
    if (bytes >= byteOffset) {
      break;
    }
    bytes += utf8Length(character);
    codeUnits += character.length;
  }
  return codeUnits;
}

function utf8Length(text: string) {
  return new TextEncoder().encode(text).length;
}
