import { pluginRegistry } from './registry'
import { MessageSearchPlugin } from './message/MessageSearchPlugin'
import { ProjectSearchPlugin } from './project/ProjectSearchPlugin'
import { SessionSearchPlugin } from './session/SessionSearchPlugin'

let registered = false

/**
 * 注册所有内置插件
 * 防止在 React Strict Mode 下重复注册
 */
export function registerBuiltinPlugins() {
  if (registered) {
    return
  }

  try {
    pluginRegistry.register(new MessageSearchPlugin())
    pluginRegistry.register(new ProjectSearchPlugin())
    pluginRegistry.register(new SessionSearchPlugin())

    registered = true
  } catch (error) {
    console.error('[Plugins] Failed to register builtin plugins:', error)
  }
}
