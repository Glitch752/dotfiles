type AnimationState = {
    timeoutId: number | null;
};

const elementStates = new WeakMap<HTMLElement, AnimationState>();

/**
* Animates overwriting an element's text content character-by-character.
* Safe to call multiple times in succession.
*/
export function animateTextChange(
    element: HTMLElement,
    newText: string,
    delay = 17,
    charactersPerUpdate = 4
): void {
    // Cancel any existing animation
    const state = elementStates.get(element);
    if(state && state.timeoutId !== null) {
        clearTimeout(state.timeoutId);
    }
    
    const oldText = element.textContent || '';
    const maxLength = Math.max(oldText.length, newText.length);
    
    const animationState: AnimationState = { timeoutId: null };
    elementStates.set(element, animationState);
    
    const step = (index: number) => {
        const prefixLength = index + 1;
        const partialNew = newText.substring(0, prefixLength);
        const partialOld = oldText.substring(prefixLength);
        element.textContent = partialNew + partialOld;
        
        if (prefixLength < maxLength) {
            animationState.timeoutId = window.setTimeout(() => step(index + charactersPerUpdate), delay);
        } else {
            element.textContent = newText;
            animationState.timeoutId = null;
        }
    };
    
    step(0);
}