<template>
  <h1>Lobby</h1>
  <template v-for="(player, index) in players" :key="index">
    <div>{{ player }}</div>
  </template>
  <button @click="readyToPlay" :disabled='buttonIsDisabled'>Ready to Play</button>
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
  computed: {
    buttonIsDisabled() {
      return this.players.length < 2
    },
  },
};
</script>

<style scoped></style>

