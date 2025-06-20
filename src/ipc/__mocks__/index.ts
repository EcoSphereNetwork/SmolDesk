export const invoke = async (cmd: string, args?: Record<string, unknown>): Promise<unknown> => {
  console.warn(`Mock invoke called with ${cmd}`)
  return null
}
