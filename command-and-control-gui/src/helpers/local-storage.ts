/**
 * Persists a value in the local storage.
 * @template T
 * @param {string} key
 * @param {T} value
 */
export function persistInLocalStorage<T>(key: string, value: T): void {
    if (typeof window !== "undefined" && typeof window.localStorage !== "undefined") {
        localStorage.setItem(key, JSON.stringify(value));
    }
}

/**
 * Retrieves a value from the local storage.
 * @template T
 * @param {string} key
 * @returns {T | null}
 */
export function getFromLocalStorage<T>(key: string): T | null {
    if (typeof window === "undefined" || typeof window.localStorage === "undefined") {
        return null;
    }

    const value = localStorage.getItem(key);

    if (value === null) {
        return null;
    }

    return JSON.parse(value);
}