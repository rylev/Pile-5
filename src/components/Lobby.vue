<template>
  <h1>Lobby</h1>
  <template v-for="(player, index) in players" :key="index">
    <div>{{ player }}</div>
  </template>
  <button @click="readyToPlay">Ready to Play</button>
</template>

<script>
export default {
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
};
</script>

<style scoped></style>

