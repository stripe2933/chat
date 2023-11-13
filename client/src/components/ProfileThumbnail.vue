<script setup>
    import { computed } from 'vue';

    const props = defineProps({
        user: {
            type: Object,
            required: true
        }
    });

    const profilePictureUrl = computed(() => {
        return `https://localhost:8443/api/user/profile_picture/${props.user.profile_picture_filename}`;
    });
</script>

<template>
    <img class="cover aspect-square rounded-full" 
        :src="profilePictureUrl">
</template>