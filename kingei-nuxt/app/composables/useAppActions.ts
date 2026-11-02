export type AppModal = 'group' | 'friend' | 'password' | null

export function useAppActions() {
  const modal = useState<AppModal>('ui.modal', () => null)
  const { canAddFriend, canCreateGroup } = useInstance()

  function open(next: Exclude<AppModal, null>) {
    if (next === 'friend' && !canAddFriend.value) {
      return
    }
    if (next === 'group' && !canCreateGroup.value) {
      return
    }
    modal.value = next
  }

  function close() {
    modal.value = null
  }

  return {
    modal,
    open,
    close,
  }
}
