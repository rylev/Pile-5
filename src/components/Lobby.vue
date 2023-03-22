<template>
  <TitleHeading />
  <div class="names">
    <template v-for="(player, index) in players" :key="index">
      <div class="player">{{ player }}</div>
    </template>
  </div>
  <button @click="readyToPlay" :disabled='buttonIsDisabled' class="button">All Players Ready?</button>
</template>

<script>
import TitleHeading from './TitleHeading.vue'
export default {
  components: {
    TitleHeading
  },
  props: {
    players: {
      type: Array,
      required: true,
    },
    readyToPlay: {
      type: Function,
      required: true
    }
  },
  data() {
    return {
      name: null
    }
  },
  computed: {
    buttonIsDisabled() {
      return this.players.length < 2
    },
  },
};
</script>

<style scoped>
.heading {
  background: #38A3A5;
  padding: 10px;
  width: 90%;
  margin: 50px auto;
  border-radius: 10px;
}

.title {
  font-family: monospace;
  color: #111111;
  font-size: 4.0rem;
}

.button {
  display: inline-block;
  outline: none;
  cursor: pointer;
  font-size: 16px;
  line-height: 20px;
  font-weight: 600;
  border-radius: 8px;
  padding: 14px 24px;
  border: none;
  background: #57CC99;
  color: #111;

  margin: 40px;
}

.button:disabled {
  background: #999;
  cursor: not-allowed;
}

.player {
  font-family: PT Sans;
  color: #111111;
  font-size: 2.0rem;
  margin: 5px;
}

.names {
  font-family: PT Sans;
  background: #C7F9CC;
  width: 50%;
  margin: 0 auto;
  border-radius: 5px;
  padding: 20px
}
</style>

