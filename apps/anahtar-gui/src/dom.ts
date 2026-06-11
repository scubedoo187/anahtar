export function requiredElement<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (!element) {
    throw new Error(`${selector} element missing`);
  }
  return element;
}

export function maybeInput(selector: string): HTMLInputElement | null {
  return document.querySelector<HTMLInputElement>(selector);
}

export function inputValue(selector: string): string {
  const input = document.querySelector<HTMLInputElement | HTMLSelectElement>(selector);
  return input?.value ?? "";
}

export function setInputValue(selector: string, value: string): void {
  const input = maybeInput(selector);
  if (input) {
    input.value = value;
  }
}

export function setDisabled(selector: string, disabled: boolean): void {
  const element = document.querySelector<HTMLInputElement | HTMLButtonElement | HTMLSelectElement>(
    selector,
  );
  if (element) {
    element.disabled = disabled;
  }
}

export function bindButton(selector: string, handler: () => Promise<void>): void {
  document.querySelector<HTMLButtonElement>(selector)?.addEventListener("click", () => {
    void handler();
  });
}

export function bindForm(selector: string, handler: () => Promise<void>): void {
  document.querySelector<HTMLFormElement>(selector)?.addEventListener("submit", (event) => {
    event.preventDefault();
    void handler();
  });
}

export function outputEl(): HTMLDivElement {
  return requiredElement<HTMLDivElement>("#command-output");
}

export function writeReportEl(): HTMLDivElement {
  return requiredElement<HTMLDivElement>("#write-report");
}

export function entryListEl(): HTMLDivElement {
  return requiredElement<HTMLDivElement>("#entry-list");
}

export function groupListEl(): HTMLDivElement {
  return requiredElement<HTMLDivElement>("#group-list");
}

export function auditFindingsEl(): HTMLDivElement {
  return requiredElement<HTMLDivElement>("#audit-findings");
}

export function detailOutputEl(): HTMLDivElement {
  return requiredElement<HTMLDivElement>("#entry-detail");
}

export function clipboardStatusEl(): HTMLDivElement {
  return requiredElement<HTMLDivElement>("#clipboard-status");
}
