<?php

use Illuminate\Database\Migrations\Migration;
use Illuminate\Database\Schema\Blueprint;
use Illuminate\Support\Facades\Schema;

return new class extends Migration
{
    public function up(): void
    {
        Schema::create('session_events', function (Blueprint $table) {
            $table->id();
            $table->foreignId('session_id')->constrained('exam_sessions')->cascadeOnDelete();
            $table->string('event_type');
            $table->string('severity')->default('info');
            $table->json('payload_json')->nullable();
            $table->dateTime('event_at');
            $table->timestamps();

            $table->index(['session_id', 'event_at']);
        });
    }

    public function down(): void
    {
        Schema::dropIfExists('session_events');
    }
};
