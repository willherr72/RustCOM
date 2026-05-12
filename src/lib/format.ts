export function formatHex(data: Uint8Array): string {
  let result = "";
  for (let i = 0; i < data.length; i += 16) {
    const chunk = data.subarray(i, Math.min(i + 16, data.length));
    result += i.toString(16).toUpperCase().padStart(4, "0") + "  ";
    for (let j = 0; j < chunk.length; j++) {
      result += chunk[j].toString(16).toUpperCase().padStart(2, "0") + " ";
      if (j === 7) result += " ";
    }
    if (chunk.length < 16) {
      for (let k = 0; k < 16 - chunk.length; k++) result += "   ";
      if (chunk.length <= 8) result += " ";
    }
    result += "  ";
    for (let j = 0; j < chunk.length; j++) {
      const b = chunk[j];
      result += (b >= 0x21 && b <= 0x7e) || b === 0x20 ? String.fromCharCode(b) : ".";
    }
    result += "\n";
  }
  return result;
}

export function stripAnsi(text: string): string {
  let out = "";
  let i = 0;
  while (i < text.length) {
    const ch = text[i];
    if (ch === "\x1b" && text[i + 1] === "[") {
      i += 2;
      while (i < text.length) {
        const c = text[i++];
        if (/[a-zA-Z]/.test(c)) break;
      }
    } else {
      out += ch;
      i++;
    }
  }
  return out;
}

const TEXT_DECODER = new TextDecoder("utf-8", { fatal: false });

export function bytesToText(data: Uint8Array, stripAnsiCodes: boolean): string {
  const raw = TEXT_DECODER.decode(data);
  return stripAnsiCodes ? stripAnsi(raw) : raw;
}
