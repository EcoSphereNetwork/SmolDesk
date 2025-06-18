export interface IWindowAPI {
  minimize(): void
  close(): void
  isFocused(): Promise<boolean>
}
