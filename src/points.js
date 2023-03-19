export function points(cardValue) {
    let pointValue;
    if (cardValue % 11 === 0) {
        pointValue = cardValue === 55 ? 6 : 5;
    } else if (cardValue % 5 === 0) {
        pointValue = cardValue % 10 === 0 ? 3 : 2;
    } else {
        pointValue = 1;
    }
    return pointValue;
}