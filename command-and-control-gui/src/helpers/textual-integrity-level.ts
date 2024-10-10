/**
 * Get textual representation of integrity level
 * @param {number} integrity_level
 * @returns {string}
 */
export function getTextualIntegrityLevel(integrity_level: number): string {
    if (integrity_level === 0x1000) {
        return "Low";
    } else if (integrity_level === 0x2000) {
        return "Medium";
    } else if (integrity_level === 0x2100) {
        return "Medium+";
    } else if (integrity_level === 0x3000) {
        return "High";
    } else if (integrity_level === 0x4000) {
        return "System";
    } else if (integrity_level === 0x5000) {
        return "Protected process";
    } else if (integrity_level === 0x7000) {
        return "Secure process";
    } else {
        return "Unknown";
    }
}