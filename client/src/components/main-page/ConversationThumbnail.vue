<script setup>
import { computed } from 'vue';
import ProfileThumbnail from '../ProfileThumbnail.vue';

const props = defineProps({
    members: {
        type: Array,
        required: true
    },
    selfUsername: {
        type: String,
        required: true
    }
});

const showingMembers = computed(() => {
    const membersExceptSelf = props.members
        .filter(member => member.username !== props.selfUsername)
        .slice(0, 4); // Take up to 4 members.
    return membersExceptSelf || [props.selfUsername];
});

const imageArrangement = computed(() => {
    const numShowingProfiles = showingMembers.value.length;

    switch (numShowingProfiles){
        case 1:
            return {
                containerStyle: [],
                elementStyle: (idx) => []
            };
        case 2:
            return {
                containerStyle: ['relative'],
                elementStyle: (idx) => {
                    if (idx === 0) {
                        return ['absolute', 'w-[70%]', 'h-[70%]'];
                    }
                    else if (idx === 1) {
                        return ['absolute', 'left-[30%]', 'top-[30%]', 'w-[70%]', 'h-[70%]'];
                    }
                }
            };
        case 3:
            return {
                containerStyle: ['relative'],
                elementStyle: (idx) => {
                    if (idx === 0) {
                        return ['absolute', 'w-[60%]', 'h-[60%]'];
                    }
                    else if (idx === 1) {
                        return ['absolute', 'left-[40%]', 'w-[60%]', 'h-[60%]'];
                    }
                    else if (idx === 2) {
                        return ['absolute', 'left-[20%]', 'top-[40%]', 'w-[60%]', 'h-[60%]'];
                    }
                }
            };
        case 4:
            return {
                containerStyle: ['grid', 'grid-cols-2'],
                elementStyle: (idx) => []
            };
    };
});
</script>

<template>
    <div :class="imageArrangement.containerStyle">
        <ProfileThumbnail
            v-for="(member, idx) in showingMembers" 
            :class="imageArrangement.elementStyle(idx)" 
            :key="member.username" 
            :user="member"/>
    </div>
</template>