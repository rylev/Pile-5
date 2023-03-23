<template>
  <div class="card" :class="{ empty: !cardValue, inert }">
    <template v-if="cardValue">
      <div class="mini-value top left">{{ cardValue }}</div>
      <div class="mini-value top right">{{ cardValue }}</div>
      <div class="mini-value bottom left">{{ cardValue }}</div>
      <div class="mini-value bottom right">{{ cardValue }}</div>
      <div class="card-background" :class="backgroundClass"></div>
      <div class="card-points">
        <template v-for="i in cardPoints" :key="i">
          <img src="../assets/atom.svg" alt="Atom Icon" class="mini-icon">
        </template>
      </div>
      <img src="../assets/atom.svg" alt="Atom Icon" class="main-icon">
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
    inert: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    let backgroundClass = {};
    backgroundClass[`background-${points(this.cardValue)}`] = !this.inert;
    return {
      backgroundClass
    }

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
  position: relative;
  background: whitesmoke;
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
}

.card-background {
  border-radius: 5px;
  position: absolute;
  width: 85%;
  height: 90%;
  z-index: 0;
}

.card.empty {
  border: 1px dashed black;
}

.card-value {
  font-family: "Bangers";
  z-index: 100;
  font-size: 3rem;
  color: #F5F4EB;
  -webkit-text-stroke: 2px black;
}

.card-points {
  position: absolute;
  top: 0;
  margin-top: 5px;
  z-index: 100;
  width: 60%;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
}

.mini-icon {
  display: block;
  margin: 0px 3px;
  width: 14px;
  height: 14px;
}

.mini-value {
  font-family: "Bangers";
  position: absolute;
  font-size: 10px;
  margin: 5px;
  z-index: 50;
}

.top {
  top: 0px;
}

.bottom {
  bottom: 0px;
}

.right {
  right: 0px;
}

.left {
  left: 0px;
}

.background-2 {
  background: #C7F9CC;
}

.background-3 {
  background: #80ED99;
}

.background-5 {
  background: #57CC99;
}

.background-6 {
  background: #38A3A5;
}

.main-icon {
  position: absolute;
  z-index: 50;
  width: 50px;
  height: 50px;
}
</style>

