<script setup>
import ProfileThumbnail from './ProfileThumbnail.vue';

defineProps({
    user: {
        type: Object,
        required: true
    }
})

</script>

<template>
    <div class="flex gap-x-2 bg-gray-800 rounded-md p-2 pr-4">
        <ProfileThumbnail class="w-12 aspect-square" :user="user"/>
        <div class="flex flex-col justify-center items-start">
            <p class="text-white text-md font-bold">{{ user.nickname }}</p>
            <div class="flex gap-x-2">
                <router-link class="text-blue-300 text-sm" to="/update_profile">Update profile</router-link>
                <router-link class="text-blue-300 text-sm" to="/logout">Logout</router-link>
            </div>
        </div>
    </div>
</template>