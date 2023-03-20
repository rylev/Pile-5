<template>
  <h1>Lobby</h1>
  <template v-for="(player, index) in players" :key="index">
    <div>{{ player }}</div>
  </template>
</template>

<script>
export default {
  props: {
    players: {
      type: Array,
      required: true,
    },
  },
  data() {
    return {
      name: null
    }
  },
};
</script>

<style scoped></style>

