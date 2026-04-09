<?php

namespace App\Filament\Resources\ExamSessions\Schemas;

use Filament\Forms\Components\TextInput;
use Filament\Schemas\Schema;

class ExamSessionForm
{
    public static function configure(Schema $schema): Schema
    {
        return $schema
            ->components([
                TextInput::make('display_name')
                    ->label('Nama')
                    ->disabled(),
                TextInput::make('class_room')
                    ->label('Kelas / Ruang')
                    ->disabled(),
                TextInput::make('status')
                    ->disabled(),
            ]);
    }
}
