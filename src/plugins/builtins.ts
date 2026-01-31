import { pluginRegistry } from './registry'
import { MessageSearchPlugin } from './message/MessageSearchPlugin'
import { ProjectSearchPlugin } from './project/ProjectSearchPlugin'
import { SessionSearchPlugin } from './session/SessionSearchPlugin'

/**
 * 注册所有内置插件
 */
export function registerBuiltinPlugins() {
  console.log('[Plugins] Registering builtin plugins...')
  
  try {
    pluginRegistry.register(new MessageSearchPlugin())
    pluginRegistry.register(new ProjectSearchPlugin())
    pluginRegistry.register(new SessionSearchPlugin())
    
    console.log('[Plugins] All builtin plugins registered successfully')
  } catch (error) {
    console.error('[Plugins] Failed to register builtin plugins:', error)
  }
}
