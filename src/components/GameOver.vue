
<template>
    <h1>Game Over</h1>
    <template v-for="([name, score], index) in Object.entries(scores)" :key="index">
        <div>{{ name }}: {{ score }}</div>
    </template>
    <button @click="startOver">Start Over</button>
</template>

<script>
export default {
    props: {
        scores: {
            type: Object,
            required: true,
        },
        startOver: {
            type: Function,
            required: true
        }
    },
};
</script>

<style scoped></style>

