<template>
  <div class="card" :class="{ empty: !cardValue }">
    <template v-if="cardValue">
      <div class="card-points">
        <template v-for="i in cardPoints" :key="i">
          <img src="../assets/cat.svg" alt="Cat Icon" class="cat-icon">
        </template>
      </div>
      <div class="card-value">{{ cardValue }}</div>
    </template>
  </div>
</template>

<script>
import { points } from '../points.js'
export default {
  props: {
    cardValue: {
      type: [Number, null],
      required: true,
    },
  },
  computed: {
    cardPoints() {
      return points(this.cardValue);
    },
  },
};
</script>

<style scoped>
.card {
  width: 100px;
  height: 150px;
  border: 1px solid black;
  border-radius: 5px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  font-size: 24px;
  font-weight: bold;
  background-color: white;
}

.card.empty {

  border: 1px dashed black;
}

.card-value {
  margin-bottom: 10px;
}

.cat-icon {
  width: 24px;
  height: 24px;
  margin-right: 4px;
}
</style>

